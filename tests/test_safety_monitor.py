"""Tests for SafetyMonitor"""
import pytest
from application.safety_monitor import SafetyMonitor
from tests.fixtures.mock_hardware import MockHardwareInterface


def test_safety_check_passes():
    """Test safety check passes with healthy battery"""
    hw = MockHardwareInterface()
    hw.initialize()

    monitor = SafetyMonitor(hw)

    # Should pass with default healthy values
    result = monitor.check_constraints({})
    assert result is True
    assert len(monitor.get_violations()) == 0


def test_safety_check_low_voltage():
    """Test safety check fails on low voltage"""
    hw = MockHardwareInterface()
    hw.initialize()
    hw.bms.simulate_low_battery()

    monitor = SafetyMonitor(hw)

    result = monitor.check_constraints({})
    assert result is False

    violations = monitor.get_violations()
    assert len(violations) > 0
    assert any("Battery low" in v for v in violations)


def test_safety_check_high_temperature():
    """Test safety check fails on high temperature"""
    hw = MockHardwareInterface()
    hw.initialize()
    hw.bms.simulate_high_temperature()

    monitor = SafetyMonitor(hw)

    result = monitor.check_constraints({})
    assert result is False

    violations = monitor.get_violations()
    assert any("Temperature critical" in v for v in violations)


def test_safety_check_high_current():
    """Test safety check fails on high current"""
    hw = MockHardwareInterface()
    hw.initialize()
    hw.bms.simulate_high_current()

    monitor = SafetyMonitor(hw)

    result = monitor.check_constraints({})
    assert result is False

    violations = monitor.get_violations()
    assert any("Current high" in v for v in violations)


def test_multiple_violations():
    """Test multiple simultaneous violations"""
    hw = MockHardwareInterface()
    hw.initialize()
    hw.bms.simulate_low_battery()
    hw.bms.simulate_high_temperature()

    monitor = SafetyMonitor(hw)

    result = monitor.check_constraints({})
    assert result is False

    violations = monitor.get_violations()
    assert len(violations) >= 2
