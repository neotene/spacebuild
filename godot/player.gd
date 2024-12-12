extends Node3D

@onready var core = $"../Core"

func _ready() -> void:
	pass

func _process(delta: float) -> void:
	if core.state == core.State.WELCOME:
		rotate((Vector3.FORWARD + Vector3.RIGHT + Vector3.UP).normalized(), 0.1 * delta)
	elif core.state == core.State.PLAYING:
		$Ship.set_visible(true)
