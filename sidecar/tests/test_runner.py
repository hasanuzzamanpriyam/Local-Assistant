from runner import Runner

def test_runner_initializes():
    runner = Runner()
    assert runner is not None
    assert hasattr(runner, "start")
    assert hasattr(runner, "handle_command")

def test_handle_ping_returns_pong():
    runner = Runner()
    result = runner.handle_command({"cmd": "ping"})
    assert result == {"cmd": "pong"}
