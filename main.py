"""Main entry point for GPM application"""
import sys
import time
import signal
from typing import Optional

from application.hardware import HardwareInterface
from application.grip_controller import GripController, GripType
from application.safety_monitor import SafetyMonitor
from application.state_machine import StateMachine, ArmState
from application.command_sequencer import CommandSequencer
from config.constants import APP_CONFIG, CONTROL_LOOP_PERIOD


class ArmController:
    """Main application orchestrator"""
   
    def __init__(self, config: dict = None):
        print("Initializing GPM...")
        
        self.state_machine = StateMachine()
        self.hardware = HardwareInterface(config)
        self.grip_controller = GripController(self.hardware)
        self.safety_monitor = SafetyMonitor(self.hardware)
        self.command_sequencer = CommandSequencer()
        
        self.running = False
        self._setup_signal_handlers()
        
    def _setup_signal_handlers(self):
        """Setup graceful shutdown handlers"""
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)
    
    def _signal_handler(self, signum, frame):
        """Handle shutdown signals"""
        print("\nShutdown signal received...")
        self.shutdown()
    
    def initialize(self) -> bool:
        """Initialize hardware and transition to IDLE state"""
        try:
            print("Initializing hardware...")
            self.hardware.initialize()
            
            # Check initial status
            status = self.hardware.get_status()
            print(f"BMS Status: {status['bms']}")
            print(f"EMG Ready: {status['emg_ready']}")
            
            # Safety check
            if not self.safety_monitor.check_constraints():
                print("Safety check failed:")
                for violation in self.safety_monitor.get_violations():
                    print(f"  - {violation}")
                self.state_machine.transition_to(ArmState.ERROR, "Initial safety check failed")
                return False
            
            self.state_machine.transition_to(ArmState.IDLE)
            print("Initialization complete")
            return True
            
        except Exception as e:
            print(f"Initialization failed: {e}")
            self.state_machine.transition_to(ArmState.ERROR, str(e))
            return False
    
    def process_emg_stream(self):
        """Main control loop: read sensors, classify gestures, execute commands"""
        print("Starting EMG processing loop...")
        self.running = True
        self.state_machine.transition_to(ArmState.ACTIVE)
        
        gesture_map = {
            1: GripType.OPEN,
            0: GripType.POWER,  # Close maps to power grip
            -1: None,  # Hold
        }
        
        loop_count = 0
        
        try:
            while self.running and self.state_machine.is_operational():
                loop_start = time.time()
                
                # Periodic safety check
                if loop_count % 100 == 0:
                    if not self.safety_monitor.check_constraints():
                        print("Safety violation detected:")
                        for violation in self.safety_monitor.get_violations():
                            print(f"  - {violation}")
                        self.state_machine.transition_to(
                            ArmState.ERROR,
                            "; ".join(self.safety_monitor.get_violations())
                        )
                        break
                
                # Read EMG data
                if self.hardware.emg.is_ready():
                    try:
                        samples = self.hardware.emg.read_buffer()
                        
                        # Process samples (simplified - take average of channels)
                        if len(samples) >= 2:
                            ch0_avg = sum(samples[::2]) / len(samples[::2])
                            ch1_avg = sum(samples[1::2]) / len(samples[1::2])
                            
                            gesture = self.hardware.emg.process_data([ch0_avg, ch1_avg])
                            
                            if gesture in gesture_map and gesture_map[gesture] is not None:
                                grip_type = gesture_map[gesture]
                                
                                if APP_CONFIG['debug_mode']:
                                    print(f"Gesture detected: {gesture} -> {grip_type.value}")
                                
                                if self.grip_controller.execute_grip(grip_type):
                                    print(f"Grip executed: {grip_type.value}")
                                else:
                                    print(f"Grip execution failed: {grip_type.value}")
                    
                    except Exception as e:
                        print(f"EMG processing error: {e}")
                
                # Maintain loop rate
                loop_count += 1
                elapsed = time.time() - loop_start
                if elapsed < CONTROL_LOOP_PERIOD:
                    time.sleep(CONTROL_LOOP_PERIOD - elapsed)
        
        except Exception as e:
            print(f"Control loop error: {e}")
            self.state_machine.transition_to(ArmState.ERROR, str(e))
        
        finally:
            self.state_machine.transition_to(ArmState.IDLE)
            print("EMG processing loop stopped")
    
    def run_demo(self):
        """Run a demo sequence of grips"""
        print("\nRunning grip demo...")
        self.state_machine.transition_to(ArmState.ACTIVE)
        
        demo_sequence = [
            (GripType.OPEN, "Opening hand"),
            (GripType.PINCH, "Pinch grip"),
            (GripType.POWER, "Power grip"),
            (GripType.REST, "Rest position"),
        ]
        
        for grip_type, description in demo_sequence:
            print(f"\n{description}...")
            
            if not self.safety_monitor.check_constraints():
                print("Safety check failed, aborting demo")
                break
            
            if self.grip_controller.execute_grip(grip_type):
                print(f"  ✓ {grip_type.value} executed")
            else:
                print(f"  ✗ {grip_type.value} failed")
            
            time.sleep(2.0)
        
        self.state_machine.transition_to(ArmState.IDLE)
        print("\nDemo complete")
    
    def shutdown(self):
        """Graceful shutdown"""
        print("Shutting down...")
        self.running = False
        self.state_machine.transition_to(ArmState.SHUTDOWN)
        
        try:
            self.hardware.shutdown()
        except Exception as e:
            print(f"Error during shutdown: {e}")
        
        print("Shutdown complete")
        sys.exit(0)


def main():
    """Main entry point"""
    print("=" * 50)
    print("GPM - Grasp Primary Module")
    print("=" * 50)
    
    controller = ArmController()
    
    if not controller.initialize():
        print("Failed to initialize, exiting")
        sys.exit(1)
    
    # Check command line arguments
    if len(sys.argv) > 1 and sys.argv[1] == "demo":
        controller.run_demo()
    else:
        print("\nStarting normal operation...")
        print("Press Ctrl+C to stop\n")
        controller.process_emg_stream()
    
    controller.shutdown()


if __name__ == "__main__":
    main()
