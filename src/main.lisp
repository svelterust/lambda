(in-package :lambda)

(defvar *editor* (make-text :size 20.0))
(text-position *editor* 10.0 10.0)
(text-color *editor* #x000000FF)
(text-set *editor* "(defun hello ()
  (format t \"Hello from Lambda!~%\")
  (values 1 2 3))")

(defvar *status* (make-text :size 14.0))
(text-position *status* 10.0 560.0)
(text-color *status* #x888888FF)
(text-set *status* "main.lisp  Ln 1 Col 0")
