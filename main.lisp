(in-package :lambda)

(defparameter *input-style*
  '(:color #xFBFBFCFF :height 75 :radius 8 :padding 20 :border-width 1.5 :border-color #xCFD5E2FF))

(defun input (label)
  (rect :style *input-style* :on-click (lambda (node) (format t "~&clicked: ~A~%" node))
    (text label :size 24 :color #x707A8CFF)))

(defun hr ()
  (rect :color #xE0E0E0FF :height 2))

(defun h1 (content)
  (text content :size 54 :color #x111111FF :weight 500))

(defun p (content)
  (text content :color #x666666FF))

(defun a (content)
  (text content :color #x3B82F6FF))

(defui *page* :gap 24 :padding 24
  (hstack :gap 24
    (input "First name")
    (input "Last name"))
  (input "Email")
  (hr)
  (vstack :align :center :gap 12
    (h1 "Lambda")
    (p "GPU-powered UI from Common Lisp.")
    (hstack :justify :end :gap 12
      (p "Cancel")
      (a "Submit"))))
