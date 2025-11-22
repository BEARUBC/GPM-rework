"""Type stubs for FSR sensor interface"""

class FsrReading:
    """FSR sensor reading"""
    fsr_id: int
    channel: int
    value: int
    pressure_detected: bool

class Fsr:
    """FSR (Force Sensitive Resistor) sensor interface"""
    
    def __init__(self) -> None:
        """Initialize the FSR sensor"""
        ...
    
    def configure(self, cs_pins: list[int], at_rest_threshold: int, pressure_threshold: int) -> None:
        """Configure FSR sensors
        
        Args:
            cs_pins: List of CS pin numbers
            at_rest_threshold: ADC value threshold for at-rest state
            pressure_threshold: ADC value threshold for pressure detection
        """
        ...
    
    def read_all(self) -> list[FsrReading]:
        """Read all FSR sensors
        
        Returns:
            List of FsrReading objects
        """
        ...
    
    def process_data(self) -> bool:
        """Process FSR data to detect pressure
        
        Returns:
            True if pressure detected on any sensor
        """
        ...
