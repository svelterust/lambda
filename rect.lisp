(in-package :lambda)

;; FFI
(cffi:defcfun ("lambda_rect_create"   make-rect)     :uint32)
(cffi:defcfun ("lambda_rect_destroy"  rect-destroy)  :void   (id :uint32))
(cffi:defcfun ("lambda_rect_position" rect-position) :void   (id :uint32) (x :number) (y :number))
(cffi:defcfun ("lambda_rect_size"     rect-size)     :void   (id :uint32) (w :number) (h :number))
(cffi:defcfun ("lambda_rect_color"    rect-color)    :void   (id :uint32) (rgba :uint32))
(cffi:defcfun ("lambda_rect_radius"   rect-radius)   :void   (id :uint32) (radius :number))
(cffi:defcfun ("lambda_rect_border"   rect-border)   :void   (id :uint32) (width :number) (rgba :uint32))
