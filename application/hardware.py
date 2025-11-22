"""Single initialization point for all hardware interfaces"""
from gpm import Maestro, Emg, Bms, Fsr
from config.constants import HARDWARE_CONFIG


class HardwareInterface:
    """Wrapper to manage all hardware initialization and lifecycle"""
   
    def __init__(self, config: dict = None):
        self.config = config or HARDWARE_CONFIG
        self.maestro = Maestro()
        self.emg = Emg()
        self.bms = Bms()
        self.fsr = Fsr()
       
    def initialize(self):
        """Initialize all hardware with config"""
        self.emg.configure(self.config.get('emg_buffer_size', 256))
        
        # Configure FSR if config available
        if 'fsr_cs_pins' in self.config:
            self.fsr.configure(
                self.config['fsr_cs_pins'],
                self.config.get('fsr_at_rest_threshold', 900),
                self.config.get('fsr_pressure_threshold', 500)
            )
        
        # Calibrate EMG if thresholds available
        if 'emg_inner_threshold' in self.config and 'emg_outer_threshold' in self.config:
            self.emg.calibrate(
                self.config['emg_inner_threshold'],
                self.config['emg_outer_threshold']
            )
       
    def get_status(self) -> dict:
        """Quick health check of all hardware"""
        bms_status = self.bms.get_status()
        return {
            'bms': {
                'voltage': bms_status.voltage,
                'current': bms_status.current,
                'temperature': bms_status.temperature,
                'is_healthy': bms_status.is_healthy,
                'charge_percentage': bms_status.charge_percentage,
            },
            'emg_ready': self.emg.is_ready(),
        }
    
    def shutdown(self):
        """Cleanup hardware resources"""
        # Move servos to rest position
        try:
            self.maestro.move_to_grip("rest")
        except Exception as e:
            print(f"Error moving to rest position: {e}")

