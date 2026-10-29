# Event Types


| Event Type | Description |
|------------|-------------|
| `telemetry.periodic` | Regular interval telemetry snapshot |
| `telemetry.on_change` | Telemetry emitted on significant change |
| `state.transition` | Operational state change |
| `safety.violation` | Safety rule violation detected |
| `safety.warning` | Safety threshold approaching |
| `safety.e_stop` | Emergency stop triggered |
| `safety.collaborative_mode_entered` | Entered collaborative operation mode |
| `safety.collaborative_mode_exited` | Exited collaborative operation mode |
| `safety.contact_detected` | Physical contact with human detected |
| `safety.force_limit_exceeded` | Force/power limit exceeded during contact |
| `safety.separation_violated` | Minimum protective separation distance violated |
| `safety.config_checksum_mismatch` | Safety configuration checksum verification failed |
| `task.started` | Task execution began |
| `task.completed` | Task execution completed |
| `task.failed` | Task execution failed |
| `task.cancelled` | Task execution cancelled |
| `navigation.goal_reached` | Navigation goal achieved |
| `navigation.path_blocked` | Path obstruction detected |
| `navigation.rerouting` | Path replanning initiated |
| `sensor.degraded` | Sensor performance degraded |
| `sensor.failed` | Sensor failure detected |
| `sensor.recovered` | Sensor recovered from failure |
| `power.low_battery` | Battery below threshold |
| `power.charging_started` | Charging session began |
| `power.charging_completed` | Charging session completed |
| `maintenance.required` | Maintenance action needed |
| `maintenance.performed` | Maintenance action completed |
| `communication.connected` | Network/fleet connection established |
| `communication.disconnected` | Network/fleet connection lost |
| `ai.decision` | AI/ML model decision trace |
| `ai.intervention` | Human override of AI decision |
| `system.startup` | System initialization |
| `system.shutdown` | System shutdown |
| `system.error` | System error occurred |
| `interaction.gesture_detected` | Human gesture recognized |
| `interaction.voice_command` | Voice command received |
| `interaction.user_present` | Human presence detected in interaction zone |
| `interaction.handover_initiated` | Object handover to/from human started |
| `interaction.handover_completed` | Object handover completed |
| `payload.loaded` | Cargo/payload loaded |
| `payload.unloaded` | Cargo/payload unloaded |
| `payload.shifted` | Payload shift detected |
| `payload.temperature_alert` | Payload temperature out of range |
| `coordination.formation_joined` | Robot joined formation |
| `coordination.formation_left` | Robot left formation |
| `coordination.resource_requested` | Shared resource requested |
| `coordination.resource_granted` | Shared resource access granted |
| `coordination.negotiation_complete` | Multi-agent negotiation completed |
| `manipulation.grasp_initiated` | Grasp attempt started |
| `manipulation.grasp_success` | Object successfully grasped |
| `manipulation.grasp_failed` | Grasp attempt failed |
| `manipulation.object_placed` | Object placement completed |
| `manipulation.tool_changed` | End-effector/tool change completed |
| `manipulation.hand_guiding_started` | Hand guiding mode initiated |
| `manipulation.hand_guiding_completed` | Hand guiding mode ended |
| `environment.door_opened` | Door/gate opened |
| `environment.elevator_called` | Elevator requested |
| `environment.elevator_entered` | Entered elevator |
| `custom.*` | Vendor/application-specific events |

---

