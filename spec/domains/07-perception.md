# 7. Perception Domain

Sensor readings and environmental perception.

```json
{
  "perception": {
    "lidar": [
      {
        "sensor_id": "lidar_front",
        "status": "ok",
        "min_range_m": 0.1,
        "max_range_m": 30.0,
        "scan_rate_hz": 15.0,
        "points_per_scan": 1800,
        "nearest_point_m": 2.3,
        "fov_deg": 270.0
      }
    ],
    "cameras": [
      {
        "sensor_id": "camera_front",
        "status": "ok",
        "resolution": "1920x1080",
        "fps": 30,
        "exposure_mode": "auto",
        "type": "rgb"
      }
    ],
    "depth_sensors": [
      {
        "sensor_id": "depth_front",
        "status": "ok",
        "min_range_m": 0.3,
        "max_range_m": 10.0,
        "type": "stereo"
      }
    ],
    "thermal_cameras": [
      {
        "sensor_id": "thermal_front",
        "status": "ok",
        "resolution": "640x480",
        "fps": 9,
        "min_temp_c": -20.0,
        "max_temp_c": 150.0,
        "hotspot_detected": false,
        "hotspot_temp_c": null
      }
    ],
    "event_cameras": [
      {
        "sensor_id": "event_cam_front",
        "status": "ok",
        "resolution": "1280x720",
        "event_rate_keps": 150.0,
        "dynamic_range_db": 120
      }
    ],
    "tactile_sensors": [
      {
        "sensor_id": "tactile_gripper",
        "status": "ok",
        "contact_detected": true,
        "pressure_kpa": 25.0,
        "slip_detected": false,
        "texture_class": "smooth"
      }
    ],
    "ultrasonic": [
      {
        "sensor_id": "us_rear_left",
        "status": "ok",
        "range_m": 1.2,
        "min_range_m": 0.02,
        "max_range_m": 4.0
      }
    ],
    "radar": [
      {
        "sensor_id": "radar_front",
        "status": "ok",
        "detections": 5,
        "max_range_m": 100.0,
        "type": "fmcw",
        "velocity_measurement": true
      }
    ],
    "imu": {
      "sensor_id": "imu_main",
      "status": "ok",
      "orientation_deg": {"roll": 0.1, "pitch": -0.2, "yaw": 45.3},
      "angular_velocity_dps": {"x": 0.0, "y": 0.0, "z": 5.5},
      "linear_acceleration_mps2": {"x": 0.1, "y": 0.0, "z": 9.81},
      "temperature_c": 28.5
    },
    "gps": {
      "sensor_id": "gps_main",
      "status": "ok",
      "fix_type": "rtk_fixed",
      "satellites_used": 18,
      "hdop": 0.8,
      "vdop": 1.2
    },
    "uwb": [
      {
        "sensor_id": "uwb_main",
        "status": "ok",
        "anchors_detected": 4,
        "position_accuracy_m": 0.1,
        "ranging_mode": "twr"
      }
    ],
    "magnetometer": {
      "sensor_id": "mag_main",
      "status": "ok",
      "heading_deg": 45.2,
      "field_strength_ut": 48.5,
      "calibration_status": "calibrated"
    },
    "barometer": {
      "sensor_id": "baro_main",
      "status": "ok",
      "pressure_hpa": 1013.25,
      "altitude_estimate_m": 185.0
    },
    "wheel_encoders": [
      {
        "sensor_id": "enc_left",
        "status": "ok",
        "ticks": 1234567,
        "velocity_rpm": 120,
        "direction": "forward"
      }
    ],
    "detections": [
      {
        "detection_id": "det-001",
        "object_id": "obj-789",
        "tracking_id": "track-001",
        "class": "pallet",
        "object_type": "pallet",
        "confidence": 0.95,
        "distance_m": 3.2,
        "bearing_deg": 15.0,
        "velocity_mps": 0.0,
        "bounding_box": {"x": 100, "y": 50, "w": 120, "h": 80},
        "dimensions_m": { "length": 1.2, "width": 0.8, "height": 0.15 },
        "pose": {
          "x_m": 4.5,
          "y_m": 2.1,
          "z_m": 0.0,
          "yaw_deg": 90.0
        },
        "frame_id": "map"
      },
      {
        "detection_id": "det-002",
        "tracking_id": "track-002",
        "class": "person",
        "confidence": 0.92,
        "distance_m": 5.1,
        "bearing_deg": -30.0,
        "velocity_mps": 1.1,
        "bounding_box": {"x": 200, "y": 50, "w": 80, "h": 200},
        "pose_estimation": {
          "skeleton_detected": true,
          "body_orientation_deg": 180
        }
      }
    ],
    "semantic_segmentation": {
      "model_id": "segnet-v2",
      "classes_detected": ["floor", "wall", "person", "forklift", "pallet"],
      "drivable_area_pct": 75.0
    },
    "point_cloud": {
      "source": "lidar_front",
      "points_count": 65000,
      "frame_id": "base_link",
      "timestamp": "2026-01-02T10:34:58.123456Z"
    },
    "environment": {
      "temperature_c": 22.0,
      "humidity_pct": 45.0,
      "light_lux": 350,
      "noise_db": 65,
      "air_quality_aqi": 42,
      "co2_ppm": 420,
      "dust_ugm3": 15.0
    }
  }
}
```

---

