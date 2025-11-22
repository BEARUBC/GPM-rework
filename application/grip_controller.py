from enum import Enum
from application.hardware import HardwareInterface


class GripState(Enum):
    IDLE = "idle"
    OPENING = "opening"
    CLOSING = "closing"
    HOLDING = "holding"


class GripType(Enum):
    REST = "rest"
    PINCH = "pinch"
    POWER = "power"
    OPEN = "open"


class GripController:
    """Orchestrates multi-step servo commands based on gesture input"""
   
    def __init__(self, hardware: HardwareInterface):
        self.hardware = hardware
        self.state = GripState.IDLE
        self.current_grip = GripType.REST
       
    def execute_grip(self, grip_type: GripType) -> bool:
        """
        Execute grip movement
       
        Args:
            grip_type: Target grip configuration
           
        Returns:
            True if execution succeeded
        """
        # Pre-check safety constraints
        if not self._check_safety():
            return False
           
        self.state = GripState.OPENING if grip_type != GripType.REST else GripState.CLOSING
       
        try:
            # Direct Rust call (no IPC!)
            self.hardware.maestro.move_to_grip(grip_type.value)
            self.current_grip = grip_type
            self.state = GripState.HOLDING
            return True
        except Exception as e:
            self.state = GripState.IDLE
            return False
           
    def _check_safety(self) -> bool:
        """Validate constraints before execution"""
        try:
            bms_status = self.hardware.bms.get_status()
           
            if not bms_status.is_healthy:
                print("BMS reports unhealthy state")
                return False
            if bms_status.voltage < 7.0:  # Critical voltage
                print(f"Battery voltage critical: {bms_status.voltage}V")
                return False
            if bms_status.temperature > 60.0:  # Thermal limit
                print(f"Battery temperature too high: {bms_status.temperature}°C")
                return False
            
            return True
        except Exception as e:
            print(f"Safety check error: {e}")
            return False
    
    def get_current_grip(self) -> GripType:
        """Get current grip type"""
        return self.current_grip
    
    def get_state(self) -> GripState:
        """Get current state"""
        return self.state
