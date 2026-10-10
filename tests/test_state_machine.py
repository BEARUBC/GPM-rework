"""Tests for StateMachine"""
import pytest
from application.state_machine import StateMachine, ArmState


def test_initial_state():
    """Test state machine starts in INITIALIZING state"""
    sm = StateMachine()
    assert sm.get_state() == ArmState.INITIALIZING


def test_valid_transition():
    """Test valid state transitions"""
    sm = StateMachine()

    # INITIALIZING -> IDLE
    result = sm.transition_to(ArmState.IDLE)
    assert result is True
    assert sm.get_state() == ArmState.IDLE

    # IDLE -> ACTIVE
    result = sm.transition_to(ArmState.ACTIVE)
    assert result is True
    assert sm.get_state() == ArmState.ACTIVE


def test_invalid_transition():
    """Test invalid state transitions are rejected"""
    sm = StateMachine()

    # Cannot go directly from INITIALIZING to ACTIVE
    result = sm.transition_to(ArmState.ACTIVE)
    assert result is False
    assert sm.get_state() == ArmState.INITIALIZING  # State unchanged


def test_is_operational():
    """Test is_operational() returns correct values"""
    sm = StateMachine()

    assert sm.is_operational() is False  # INITIALIZING

    sm.transition_to(ArmState.IDLE)
    assert sm.is_operational() is True  # IDLE

    sm.transition_to(ArmState.ACTIVE)
    assert sm.is_operational() is True  # ACTIVE

    sm.transition_to(ArmState.ERROR)
    assert sm.is_operational() is False  # ERROR


def test_error_state_transition():
    """Test transition to ERROR state"""
    sm = StateMachine()
    sm.transition_to(ArmState.IDLE)

    # Can transition to ERROR from any state
    result = sm.transition_to(ArmState.ERROR)
    assert result is True
    assert sm.get_state() == ArmState.ERROR


def test_shutdown_transition():
    """Test transition to SHUTDOWN state"""
    sm = StateMachine()
    sm.transition_to(ArmState.IDLE)

    # Can transition to SHUTDOWN from most states
    result = sm.transition_to(ArmState.SHUTDOWN)
    assert result is True
    assert sm.get_state() == ArmState.SHUTDOWN
