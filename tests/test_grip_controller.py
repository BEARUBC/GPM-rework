"""Tests for GripController using mock hardware"""
import pytest
from application.grip_controller import GripController, GripType, GripState
from tests.fixtures.mock_hardware import MockHardwareInterface


def test_grip_execution():
    """Test basic grip execution"""
    hw = MockHardwareInterface()
    hw.initialize()

    controller = GripController(hw)

    # Execute grip
    result = controller.execute_grip(GripType.PINCH)
    assert result is True

    # Verify maestro was called
    assert ('move_to_grip', 'pinch') in hw.maestro.call_history
    assert controller.current_grip == GripType.PINCH


def test_safety_check_low_battery():
    """Test safety check blocks on low battery"""
    hw = MockHardwareInterface()
    hw.initialize()
    hw.bms.simulate_low_battery()  # Set voltage to 6.5V

    controller = GripController(hw)

    # Should fail safety check
    result = controller.execute_grip(GripType.POWER)
    assert result is False


def test_grip_state_transitions():
    """Test state transitions during grip execution"""
    hw = MockHardwareInterface()
    hw.initialize()

    controller = GripController(hw)
    assert controller.state == GripState.IDLE

    controller.execute_grip(GripType.OPEN)
    assert controller.state == GripState.HOLDING

    controller.execute_grip(GripType.REST)
    assert controller.state == GripState.HOLDING


def test_multiple_grip_changes():
    """Test multiple sequential grip changes"""
    hw = MockHardwareInterface()
    hw.initialize()

    controller = GripController(hw)

    grips = [GripType.PINCH, GripType.POWER, GripType.OPEN, GripType.REST]
    for grip in grips:
        result = controller.execute_grip(grip)
        assert result is True
        assert controller.current_grip == grip


def test_get_current_grip():
    """Test getting current grip type"""
    hw = MockHardwareInterface()
    hw.initialize()

    controller = GripController(hw)

    assert controller.get_current_grip() == GripType.REST

    controller.execute_grip(GripType.PINCH)
    assert controller.get_current_grip() == GripType.PINCH


def test_get_state():
    """Test getting current state"""
    hw = MockHardwareInterface()
    hw.initialize()

    controller = GripController(hw)

    assert controller.get_state() == GripState.IDLE

    controller.execute_grip(GripType.POWER)
    assert controller.get_state() == GripState.HOLDING
