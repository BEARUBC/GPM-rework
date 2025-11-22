from application.hardware import HardwareInterface


class SafetyMonitor:
    """Continuous safety constraint validation"""
   
    CRITICAL_VOLTAGE = 7.0
    MAX_TEMPERATURE = 60.0
    MAX_CURRENT = 10.0  # Amps
   
    def __init__(self, hardware: HardwareInterface):
        self.hardware = hardware
        self.violations = []
       
    def check_constraints(self, desired_command: dict = None) -> bool:
        """
        Validate that desired command respects safety constraints
       
        Args:
            desired_command: Command to validate (optional)
           
        Returns:
            True if safe to execute
        """
        try:
            bms_status = self.hardware.bms.get_status()
        except Exception as e:
            self.violations = [f"BMS read error: {e}"]
            return False
       
        violations = []
       
        if bms_status.voltage < self.CRITICAL_VOLTAGE:
            violations.append(f"Battery low: {bms_status.voltage}V")
           
        if bms_status.temperature > self.MAX_TEMPERATURE:
            violations.append(f"Temperature critical: {bms_status.temperature}°C")
           
        if bms_status.current > self.MAX_CURRENT:
            violations.append(f"Current high: {bms_status.current}A")
        
        if not bms_status.is_healthy:
            violations.append("BMS reports unhealthy state")
           
        self.violations = violations
        return len(violations) == 0
       
    def get_violations(self) -> list[str]:
        """Return list of current safety violations"""
        return self.violations.copy()
    
    def is_safe(self) -> bool:
        """Quick safety check"""
        return self.check_constraints()
