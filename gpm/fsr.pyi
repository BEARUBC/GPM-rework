"""Type stubs for FSR sensor interface"""
from typing import Optional

class FsrReading:
    """FSR sensor reading"""
    fsr_id: int
    channel: int
    value: int
    pressure_detected: bool

class Fsr:
    """FSR (Force Sensitive Resistor) sensor interface"""
    
    def __init__(self, cs_pin: Optional[int] = None) -> None:
        """Initialize the FSR sensor (default CS: GPIO 7)"""
        ...
    
    def configure(self, at_rest_threshold: int, pressure_threshold: int) -> None:
        """Configure FSR sensors
        
        Args:
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
