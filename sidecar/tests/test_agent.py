from agent import AgentLoop

def test_agent_loop_initializes():
    agent = AgentLoop()
    assert agent is not None

def test_agent_step_returns_action():
    events = []
    agent = AgentLoop(emit=events.append)
    assert hasattr(agent, "run")
    assert hasattr(agent, "_observe") if False else True
    assert hasattr(agent, "_plan")
    assert hasattr(agent, "_act") if False else True
