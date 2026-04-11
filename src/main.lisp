(in-package :lambda)

(declaim (type single-float *x* *y*))
(defvar *x* 100.0)
(defvar *y* 100.0)
(defvar *size* 64.0)
(defvar *color* #x000000FF)

(with-input (type key mods x y)
  (when (eq type :key-down)
    (case key
      (:up    (decf *y* 5.0))
      (:down  (incf *y* 5.0))
      (:left  (decf *x* 5.0))
      (:right (incf *x* 5.0)))))

(with-update
  (rect *x* *y* *size* *size* *color*))
