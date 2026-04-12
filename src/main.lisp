(in-package :lambda)

;; Dark background rect
(defvar *bg* (make-rect))
(rect-position *bg* 0.0 0.0)
(rect-size *bg* 400.0 200.0)
(rect-color *bg* #x1E1E1EFF)

;; Title text on top
(defvar *lambda* (make-text :size 100.0))
(text-position *lambda* 15.0 20.0)
(text-color *lambda* #xFFFFFFFF)
(text-set *lambda* "Lambda")
