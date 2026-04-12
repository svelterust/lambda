(in-package :lambda)

;; Input field
(defparameter *bg* (make-rect))
(rect-position *bg* 20.0 20.0)
(rect-size *bg* 500.0 75.0)
(rect-color *bg* #xFBFBFCFF)
(rect-radius *bg* 8.0)
(rect-border *bg* 1.5 #xCFD5E2FF)

;; Label
(defparameter *lambda* (make-text :size 24.0)) 
(text-position *lambda* 40.0 40.0)
(text-color *lambda* #x707A8CFF) 
(text-set *lambda* "First name")

;; Image
(defparameter *slint* (make-image "/home/odd/downloads/2026-04-10-142010.png"))
(image-position *slint* 20.0 120.0)

