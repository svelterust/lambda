(in-package :lambda)

;; Input field
(defparameter *bg* (make-rect))
(rect-position *bg* 20 20)
(rect-size *bg* 500 75)
(rect-color *bg* #xFBFBFCFF)
(rect-radius *bg* 8)
(rect-border *bg* 1.5 #xCFD5E2FF)

;; Label
(defparameter *lambda* (make-text 24 33.6))
(text-position *lambda* 40 40)
(text-color *lambda* #x707A8CFF)
(text-set *lambda* "First name")

;; Image
(defparameter *slint* (make-image "/home/odd/downloads/slint.png"))
(image-position *slint* 20 120)

(defparameter *cat* (make-image "/home/odd/downloads/cat.jpg"))
(image-position *cat* 200 370)

;; SVG
(defparameter *circle* (make-image "/home/odd/downloads/circle.svg"))
(image-position *circle* 20 120)

(defparameter *star* (make-image "/home/odd/downloads/star.svg"))
(image-position *star* 160 120)

(defparameter *lam* (make-image "/home/odd/downloads/lambda.svg"))
(image-position *lam* 300 120)
