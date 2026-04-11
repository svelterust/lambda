(in-package :lambda)

(with-input (type key mods x y)
  (case type
    (:mouse-down (format t "CLICK ~A X ~A Y ~A~%" key x y))))

(with-scene
  (rect 30.0  30.0  200.0 120.0 #xE63946FF)
  (rect 260.0 30.0  200.0 120.0 #x2A9D8FFF)
  (rect 30.0  180.0 430.0 80.0  #xE9C46AFF)
  (rect 30.0  290.0 200.0 200.0 #x264653FF)
  (rect 260.0 290.0 200.0 200.0 #xF4A261FF))
