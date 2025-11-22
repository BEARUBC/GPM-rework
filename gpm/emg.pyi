"""Type stubs for EMG sensor interface"""

class Emg:
    """EMG (Electromyography) sensor interface"""
    
    def __init__(self) -> None:
        """Initialize the EMG sensor"""
        ...
    
    def configure(self, buffer_size: int) -> None:
        """Configure EMG buffer size
        
        Args:
            buffer_size: Number of samples to buffer
        """
        ...
    
    def read_buffer(self) -> list[int]:
        """Read buffer of EMG samples
        
        Returns:
            List of ADC values from both channels
        """
        ...
    
    def is_ready(self) -> bool:
        """Check if EMG is ready to read
        
        Returns:
            True if ready
        """
        ...
    
    def get_latest_samples(self) -> list[int]:
        """Get latest samples without reading new data
        
        Returns:
            List of most recent samples
        """
        ...
    
    def calibrate(self, inner_threshold: float, outer_threshold: float) -> None:
        """Calibrate EMG thresholds
        
        Args:
            inner_threshold: Threshold for inner channel
            outer_threshold: Threshold for outer channel
        """
        ...
    
    def process_data(self, values: list[float]) -> int:
        """Process EMG values to detect gesture
        
        Args:
            values: List of two float values [channel_0, channel_1]
            
        Returns:
            -1 (hold), 0 (close), or 1 (open)
        """
        ...
