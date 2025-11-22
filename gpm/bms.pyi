"""Type stubs for BMS interface"""

class BmsStatus:
    """Battery Management System status"""
    voltage: float
    current: float
    temperature: float
    is_healthy: bool
    charge_percentage: float

class Bms:
    """BMS (Battery Management System) interface"""
    
    def __init__(self) -> None:
        """Initialize the BMS"""
        ...
    
    def get_status(self) -> BmsStatus:
        """Get current BMS status
        
        Returns:
            BmsStatus object with voltage, current, temperature, health status
        """
        ...
    
    def update(self) -> None:
        """Update BMS readings"""
        ...
