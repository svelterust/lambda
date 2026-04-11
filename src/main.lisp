(in-package :lambda)

(defparameter *x* 700.0)
(defparameter *y* 500.0)

(with-input (type key mods x y)
  (when (eq type :key-down)
    (case key
      (:up    (decf *y* 10.0))
      (:down  (incf *y* 10.0))
      (:left  (decf *x* 10.0))
      (:right (incf *x* 10.0)))))

(with-draw
  (rect *x* *y* 64.0 64.0 #xE63946FF))
