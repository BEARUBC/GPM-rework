"""Command-line interface for GPM"""
import argparse
import sys
from main import ArmController


def main():
    """CLI entry point"""
    parser = argparse.ArgumentParser(
        description="GPM - Grasp Primary Module Command Line Interface"
    )
    
    parser.add_argument(
        'mode',
        choices=['demo', 'run', 'status', 'calibrate'],
        help='Operation mode'
    )
    
    parser.add_argument(
        '--debug',
        action='store_true',
        help='Enable debug output'
    )
    
    args = parser.parse_args()
    
    controller = ArmController()
    
    if not controller.initialize():
        print("Failed to initialize")
        sys.exit(1)
    
    if args.mode == 'demo':
        controller.run_demo()
    elif args.mode == 'run':
        print("Starting normal operation. Press Ctrl+C to stop.")
        controller.process_emg_stream()
    elif args.mode == 'status':
        status = controller.hardware.get_status()
        print("\n=== GPM Status ===")
        print(f"BMS:")
        for key, value in status['bms'].items():
            print(f"  {key}: {value}")
        print(f"\nEMG Ready: {status['emg_ready']}")
        print()
    elif args.mode == 'calibrate':
        print("Calibration mode not yet implemented")
    
    controller.shutdown()


if __name__ == "__main__":
    main()
