(in-package :lambda)

;;; FFI
(cffi:defcfun ("lambda_text_create"   make-text)     :uint32  (font-size :number) (line-height :number))
(cffi:defcfun ("lambda_text_destroy"  text-destroy)  :void    (id :uint32))
(cffi:defcfun ("lambda_text_set"      %text-set)     :void    (id :uint32) (ptr :pointer) (len :uint32))
(cffi:defcfun ("lambda_text_position" text-position)  :void   (id :uint32) (x :number) (y :number))
(cffi:defcfun ("lambda_text_bounds"   text-bounds)   :void    (id :uint32) (left :int32) (top :int32) (right :int32) (bottom :int32))
(cffi:defcfun ("lambda_text_color"    text-color)    :void    (id :uint32) (rgba :uint32))
(cffi:defcfun ("lambda_text_metrics"  text-metrics)  :void    (id :uint32) (font-size :number) (line-height :number))

(defun text-set (id string)
  (let ((octets (sb-ext:string-to-octets string :external-format :utf-8)))
    (sb-sys:with-pinned-objects (octets)
      (%text-set id (sb-sys:vector-sap octets) (length octets)))))
