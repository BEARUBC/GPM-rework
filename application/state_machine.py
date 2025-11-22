"""State machine for arm state management"""
from enum import Enum
from typing import Optional


class ArmState(Enum):
    """States of the prosthetic arm"""
    INITIALIZING = "initializing"
    IDLE = "idle"
    ACTIVE = "active"
    CALIBRATING = "calibrating"
    ERROR = "error"
    SHUTDOWN = "shutdown"


class StateTransition:
    """Represents a state transition with validation"""
    
    VALID_TRANSITIONS = {
        ArmState.INITIALIZING: [ArmState.IDLE, ArmState.ERROR],
        ArmState.IDLE: [ArmState.ACTIVE, ArmState.CALIBRATING, ArmState.SHUTDOWN, ArmState.ERROR],
        ArmState.ACTIVE: [ArmState.IDLE, ArmState.ERROR, ArmState.SHUTDOWN],
        ArmState.CALIBRATING: [ArmState.IDLE, ArmState.ERROR],
        ArmState.ERROR: [ArmState.IDLE, ArmState.SHUTDOWN],
        ArmState.SHUTDOWN: [],
    }
    
    @staticmethod
    def is_valid(from_state: ArmState, to_state: ArmState) -> bool:
        """Check if transition is valid"""
        return to_state in StateTransition.VALID_TRANSITIONS.get(from_state, [])


class StateMachine:
    """Manages state transitions for the prosthetic arm"""
    
    def __init__(self):
        self.current_state = ArmState.INITIALIZING
        self.previous_state: Optional[ArmState] = None
        self.error_message: Optional[str] = None
    
    def transition_to(self, new_state: ArmState, error_message: Optional[str] = None) -> bool:
        """
        Attempt to transition to a new state
        
        Args:
            new_state: Target state
            error_message: Optional error message if transitioning to ERROR state
            
        Returns:
            True if transition succeeded
        """
        if not StateTransition.is_valid(self.current_state, new_state):
            print(f"Invalid transition: {self.current_state} -> {new_state}")
            return False
        
        self.previous_state = self.current_state
        self.current_state = new_state
        
        if new_state == ArmState.ERROR:
            self.error_message = error_message or "Unknown error"
        else:
            self.error_message = None
        
        print(f"State transition: {self.previous_state} -> {self.current_state}")
        return True
    
    def get_state(self) -> ArmState:
        """Get current state"""
        return self.current_state
    
    def is_operational(self) -> bool:
        """Check if arm is operational"""
        return self.current_state in [ArmState.IDLE, ArmState.ACTIVE]
    
    def get_error_message(self) -> Optional[str]:
        """Get current error message if in ERROR state"""
        return self.error_message
