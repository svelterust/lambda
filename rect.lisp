(in-package :lambda)

;; FFI
(defcfun "rect_create"       make-rect          :uint32)
(defcfun "rect_destroy"      rect-destroy       :void   (id :uint32))
(defcfun "rect_position"     rect-position      :void   (id :uint32) (x :number) (y :number))
(defcfun "rect_size"         rect-size          :void   (id :uint32) (w :number) (h :number))
(defcfun "rect_color"        rect-color         :void   (id :uint32) (rgba :uint32))
(defcfun "rect_radius"       rect-radius        :void   (id :uint32) (radius :number))
(defcfun "rect_border"       rect-border        :void   (id :uint32) (width :number) (rgba :uint32))
(defcfun "rect_border_width" rect-border-width  :void   (id :uint32) (width :number))
(defcfun "rect_border_color" rect-border-color  :void   (id :uint32) (rgba :uint32))
