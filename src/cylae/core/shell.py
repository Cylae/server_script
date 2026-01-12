import subprocess
import logging
import shlex

logger = logging.getLogger(__name__)

def run_command(command, check=True, capture_output=True, input=None, env=None, shell=False):
    """
    Runs a shell command.

    Args:
        command (list or str): The command to run.
        check (bool): Whether to raise an exception on non-zero exit code.
        capture_output (bool): Whether to capture stdout/stderr.
        input (str): Input string to pass to stdin.
        env (dict): Environment variables.
        shell (bool): Whether to run through shell (use with caution).

    Returns:
        subprocess.CompletedProcess: The result object.
    """
    if isinstance(command, str) and not shell:
        command = shlex.split(command)

    cmd_str = command if isinstance(command, str) else ' '.join(command)
    logger.debug(f"Running command: {cmd_str}")

    try:
        result = subprocess.run(
            command,
            check=check,
            capture_output=capture_output,
            text=True,
            input=input,
            env=env,
            shell=shell
        )
        if capture_output and result.stdout:
            logger.debug(f"STDOUT: {result.stdout.strip()}")
        if capture_output and result.stderr:
            logger.debug(f"STDERR: {result.stderr.strip()}")

        return result
    except subprocess.CalledProcessError as e:
        logger.error(f"Command failed: {' '.join(command)}")
        logger.error(f"Exit code: {e.returncode}")
        if e.stdout:
            logger.error(f"STDOUT: {e.stdout.strip()}")
        if e.stderr:
            logger.error(f"STDERR: {e.stderr.strip()}")
        raise
