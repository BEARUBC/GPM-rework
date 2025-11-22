"""Type stubs for Maestro servo controller"""

class Maestro:
    """Maestro servo controller interface"""
    
    def __init__(self) -> None:
        """Initialize the Maestro controller"""
        ...
    
    def set_target(self, channel: int, pwm_value: int) -> None:
        """Set target PWM for a servo channel
        
        Args:
            channel: Servo channel (0-5)
            pwm_value: PWM value (typically 1000-2000)
        """
        ...
    
    def move_to_grip(self, grip_type: str) -> None:
        """Move to predefined grip type
        
        Args:
            grip_type: One of "rest", "pinch", "power", "open"
        """
        ...
    
    def current_pwm(self, channel: int) -> int:
        """Get current PWM value for channel
        
        Args:
            channel: Servo channel (0-5)
            
        Returns:
            Current PWM value
        """
        ...