extends Node

func _ready() -> void:
	set_process_input(true) 

func _process(delta: float) -> void:
	if Input.is_key_pressed(KEY_Z):
		self.position += Vector3.FORWARD * 10 * delta
	(get_node("Camera3D") as Camera3D).rotate((Vector3.FORWARD + Vector3.RIGHT + Vector3.UP).normalized(), 0.1 * delta)
