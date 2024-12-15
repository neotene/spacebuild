extends Node3D

@onready var core = get_tree().get_first_node_in_group("core")

func _ready() -> void:
	pass

func _process(delta: float) -> void:
	if core.state == core.State.WELCOME:
		$Ship.set_visible(false)
		rotate((Vector3.FORWARD + Vector3.RIGHT + Vector3.UP).normalized(), 0.1 * delta)
	elif core.state == core.State.PLAYING_SOLO || core.state == core.State.PLAYING_ONLINE:
		$Ship.set_visible(true)
