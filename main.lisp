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
(defparameter *slint* (make-image "/home/odd/downloads/slint.png"))
(image-position *slint* 20.0 120.0)

(defparameter *cat* (make-image "/home/odd/downloads/cat.jpg"))
(image-position *cat* 200.0 370.0)

;; SVG
(defparameter *circle* (make-image "/home/odd/downloads/circle.svg"))
(image-position *circle* 20.0 120.0)

(defparameter *star* (make-image "/home/odd/downloads/star.svg"))
(image-position *star* 160.0 120.0)

(defparameter *lam* (make-image "/home/odd/downloads/lambda.svg"))
(image-position *lam* 300.0 120.0)
