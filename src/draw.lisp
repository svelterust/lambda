(in-package :lambda)

(cffi:defcfun ("lambda_buf_ptr"       %buf-ptr)       :pointer)
(cffi:defcfun ("lambda_buf_set_count" %buf-set-count) :void (n :uint32))

(cffi:defcstruct draw-cmd
  (x     :float)
  (y     :float)
  (w     :float)
  (h     :float)
  (color :uint32))

;; Draw commands
(defparameter *buf* (%buf-ptr))
(defvar *idx* 0)

(defun clear ()
  "Reset the draw command index to zero."
  (setf *idx* 0))

(defun rect (rx ry rw rh rc)
  "Write a rectangle draw command to the shared buffer."
  (declare (type single-float rx ry rw rh)
           (type (unsigned-byte 32) rc))
  (let ((ptr (cffi:inc-pointer *buf* (* *idx* (cffi:foreign-type-size '(:struct draw-cmd))))))
    (cffi:with-foreign-slots ((x y w h color) ptr (:struct draw-cmd))
      (setf x rx y ry w rw h rh color rc)))
  (incf *idx*))

(defun flush ()
  "Publish the current draw commands to the renderer."
  (%buf-set-count *idx*))

(defmacro with-scene (&body body)
  "Clear the buffer, evaluate body, then flush."
  `(progn (clear) ,@body (flush)))
