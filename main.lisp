(in-package :lambda)

(defun input-field (label)
  (rect (:color #xFBFBFCFF :height 75 :radius 8 :padding 20
         :border-width 1.5 :border-color #xCFD5E2FF
         :on-click (lambda (node) (format t "~&clicked: ~a~%" label)))
    (text label (:size 24 :color #x707A8CFF))))

(defun divider ()
  (rect (:color #xE0E0E0FF :height 2)))

(defui *page* (:gap 24 :padding 16)
  (hstack (:gap 24)
    (input-field "First name")
    (input-field "Last name"))
  (input-field "Email")
  (divider)
  (vstack (:align :center :gap 8)
    (text "Lambda" (:size 54 :color #x111111FF :weight 500))
    (text "GPU-powered UI from Common Lisp." (:size 20 :color #x666666FF)))
  (hstack (:justify :end :gap 12)
    (text "Cancel" (:size 18 :color #x666666FF))
    (text "Submit" (:size 18 :color #x3B82F6FF))))

(handle-input (type key mods x y)
  (when (eq type :mouse-down)
    (let ((node (node-at (ui-root *page*) x y :on-click)))
      (when node
        (funcall (getf (node-styles node) :on-click) node)))))
