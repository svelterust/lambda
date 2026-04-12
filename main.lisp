(in-package :lambda)

(defparameter *input-style*
  '(:color #xFBFBFCFF :height 75 :radius 8
    :border-width 1.5 :border-color #xCFD5E2FF :padding 25))

(defun input (label)
  `(rect ,*input-style*
		 (text ,label (:size 24 :color #x707A8CFF))))

(defun divider ()
  `(rect (:color #xE0E0E0FF :height 2)))

(defui *page* (:gap 24 :padding 16)
  (input "First name")
  (input "Last name")
  (divider)
  (text "Layout engine working!" (:size 24 :color #x707A8CFF)))
