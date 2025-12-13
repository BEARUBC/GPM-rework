"""Centralized logging configuration for GPM application"""
from loguru import logger
import sys


def setup_logging(log_level="INFO", log_file=None, console=True):
    """Configure loguru logger for GPM application

    Args:
        log_level: One of DEBUG, INFO, WARNING, ERROR, CRITICAL
        log_file: Optional file path for log output (None to disable)
        console: Whether to log to console (default: True)

    Returns:
        logger: Configured loguru logger instance
    """
    # Remove default handler
    logger.remove()

    # Console handler with color and formatting
    if console:
        logger.add(
            sys.stderr,
            format="<green>{time:YYYY-MM-DD HH:mm:ss.SSS}</green> | <level>{level: <8}</level> | <cyan>{name}</cyan>:<cyan>{function}</cyan>:<cyan>{line}</cyan> - <level>{message}</level>",
            level=log_level,
            colorize=True,
        )

    # File handler (if specified)
    if log_file:
        logger.add(
            log_file,
            format="{time:YYYY-MM-DD HH:mm:ss.SSS} | {level: <8} | {name}:{function}:{line} - {message}",
            level=log_level,
            rotation="10 MB",  # Rotate when file reaches 10MB
            retention="7 days",  # Keep logs for 7 days
            compression="zip",  # Compress rotated logs
            enqueue=True,  # Thread-safe logging
        )

    logger.info(f"Logging configured: level={log_level}, file={log_file}, console={console}")

    return logger
