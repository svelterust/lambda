(in-package :lambda)

;;; FFI
(cffi:defcfun ("lambda_rect_create"   %rect-create)   :uint32)

(cffi:defcfun ("lambda_rect_destroy"  %rect-destroy)  :void
  (id :uint32))

(cffi:defcfun ("lambda_rect_position" %rect-position) :void
  (id :uint32) (x :float) (y :float))

(cffi:defcfun ("lambda_rect_size"     %rect-size)     :void
  (id :uint32) (w :float) (h :float))

(cffi:defcfun ("lambda_rect_color"    %rect-color)    :void
  (id :uint32) (rgba :uint32))

(cffi:defcfun ("lambda_rect_radius"  %rect-radius)  :void
  (id :uint32) (radius :float))

(cffi:defcfun ("lambda_rect_border"  %rect-border)  :void
  (id :uint32) (width :float) (rgba :uint32))

;;; Rect
(defun make-rect ()
  "Create a new rectangle. Returns an integer ID."
  (%rect-create))

(defun rect-destroy (id)
  "Destroy a rectangle."
  (declare (type (unsigned-byte 32) id))
  (%rect-destroy id))

(defun rect-position (id x y)
  "Set the screen position of a rectangle."
  (declare (type (unsigned-byte 32) id)
           (type single-float x y))
  (%rect-position id x y))

(defun rect-size (id w h)
  "Set the width and height of a rectangle."
  (declare (type (unsigned-byte 32) id)
           (type single-float w h))
  (%rect-size id w h))

(defun rect-color (id rgba)
  "Set the fill color as #xRRGGBBAA."
  (declare (type (unsigned-byte 32) id rgba))
  (%rect-color id rgba))

(defun rect-radius (id radius)
  "Set the corner radius of a rectangle."
  (declare (type (unsigned-byte 32) id)
           (type single-float radius))
  (%rect-radius id radius))

(defun rect-border (id width rgba)
  "Set the border width and color as #xRRGGBBAA."
  (declare (type (unsigned-byte 32) id rgba)
           (type single-float width))
  (%rect-border id width rgba))
