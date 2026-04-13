(in-package :lambda)

(defparameter *input-style*
  '(:color #xFBFBFCFF :height 75 :radius 8 :border-width 1.5 :border-color #xCFD5E2FF :padding 25))

(defun input (label)
  (rect *input-style*
    (text label (:size 24 :color #x707A8CFF))))

(defun divider ()
  (rect (:color #xE0E0E0FF :height 2)))

(defui *page* (:gap 24 :padding 16)
  (hstack (:gap 24)
    (input "First name")
    (input "Last name"))
  (input "Email")
  (divider)
  (vstack (:align :center :gap 8)
    (text "Lambda" (:size 48 :color #x111111FF))
    (text "GPU-powered UI from Common Lisp." (:size 18 :color #x666666FF)))
  (hstack (:justify :end :gap 12)
    (text "Cancel" (:size 16 :color #x666666FF))
    (text "Submit" (:size 16 :color #x3B82F6FF))))
