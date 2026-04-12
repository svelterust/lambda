(in-package :lambda)

;; Dark rounded background rect
(defparameter *bg* (make-rect))
(rect-position *bg* 20.0 20.0)
(rect-size *bg* 400.0 100.0)
(rect-color *bg* #x1E1E1EFF)
(rect-radius *bg* 16.0)
(rect-destroy *bg*)

;; Title text on top
(defparameter *lambda* (make-text :size 100.0))
(text-set *lambda* "Lambda")
(text-position *lambda* 50.0 50.0)
(text-destroy *lambda*)
