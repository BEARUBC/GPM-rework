"""Command sequencer for multi-step commands"""
from typing import List, Callable, Optional
from dataclasses import dataclass
import time


@dataclass
class Command:
    """Represents a single command in a sequence"""
    name: str
    action: Callable[[], bool]
    delay_after: float = 0.0  # Seconds to wait after execution
    
    def execute(self) -> bool:
        """Execute the command"""
        try:
            result = self.action()
            if result and self.delay_after > 0:
                time.sleep(self.delay_after)
            return result
        except Exception as e:
            print(f"Command '{self.name}' failed: {e}")
            return False


class CommandSequence:
    """A sequence of commands to execute"""
    
    def __init__(self, name: str, commands: List[Command]):
        self.name = name
        self.commands = commands
        self.current_index = 0
        self.completed = False
    
    def execute_all(self) -> bool:
        """Execute all commands in sequence"""
        print(f"Executing sequence: {self.name}")
        
        for i, command in enumerate(self.commands):
            self.current_index = i
            print(f"  [{i+1}/{len(self.commands)}] Executing: {command.name}")
            
            if not command.execute():
                print(f"  Sequence '{self.name}' failed at step {i+1}")
                return False
        
        self.completed = True
        print(f"Sequence '{self.name}' completed successfully")
        return True
    
    def reset(self):
        """Reset sequence to beginning"""
        self.current_index = 0
        self.completed = False


class CommandSequencer:
    """Manages and executes command sequences"""
    
    def __init__(self):
        self.sequences: dict[str, CommandSequence] = {}
        self.current_sequence: Optional[CommandSequence] = None
    
    def register_sequence(self, sequence: CommandSequence):
        """Register a named sequence"""
        self.sequences[sequence.name] = sequence
    
    def execute_sequence(self, sequence_name: str) -> bool:
        """Execute a registered sequence by name"""
        if sequence_name not in self.sequences:
            print(f"Unknown sequence: {sequence_name}")
            return False
        
        self.current_sequence = self.sequences[sequence_name]
        self.current_sequence.reset()
        return self.current_sequence.execute_all()
    
    def create_grip_sequence(self, grip_controller, grip_type: str) -> CommandSequence:
        """
        Create a command sequence for grip execution
        
        Args:
            grip_controller: GripController instance
            grip_type: Target grip type
            
        Returns:
            CommandSequence for grip execution
        """
        from application.grip_controller import GripType
        
        target_grip = GripType[grip_type.upper()]
        
        commands = [
            Command(
                name="Safety check",
                action=lambda: grip_controller._check_safety(),
                delay_after=0.1
            ),
            Command(
                name=f"Move to {grip_type}",
                action=lambda: grip_controller.execute_grip(target_grip),
                delay_after=0.5
            ),
        ]
        
        return CommandSequence(name=f"grip_{grip_type}", commands=commands)
