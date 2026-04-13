(in-package :lambda)

;; FFI
(cffi:defcfun ("lambda_window_width"  window-width)  :uint32)
(cffi:defcfun ("lambda_window_height" window-height) :uint32)

(defstruct node
  "One element in the UI tree."
  type          ; :vstack :hstack :rect :text :image
  content       ; string or nil (text content, image path)
  styles        ; plist (:color #xFF0000FF :size 24 ...)
  children      ; list of child nodes
  id            ; u32 GPU element ID, nil for layout-only nodes
  x y width height) ; computed by layout

(defstruct ui
  "Root node and name lookup table."
  root          ; root node (implicit vstack)
  names)        ; hash table: keyword -> node

(defun flatten-children (forms)
  "Flatten one level of nesting and remove nils from children."
  (mapcan (lambda (f)
            (cond ((node-p f) (list f))
                  ((listp f) (remove-if-not #'node-p f))))
          forms))

(defun expand-styles (styles)
  "Expand styles form: inline plist becomes (list ...), otherwise passed through."
  (if (and (listp styles) (keywordp (car styles)))
      `(list ,@styles)
      styles))

;; Element macros
(defmacro vstack (styles &body children)
  `(make-node :type :vstack
              :styles ,(expand-styles styles)
              :children (flatten-children (list ,@children))))

(defmacro hstack (styles &body children)
  `(make-node :type :hstack
              :styles ,(expand-styles styles)
              :children (flatten-children (list ,@children))))

(defmacro rect (styles &body children)
  `(make-node :type :rect
              :styles ,(expand-styles styles)
              :children (flatten-children (list ,@children))))

(defmacro text (content &optional styles)
  `(make-node :type :text
              :content ,content
              :styles ,(expand-styles styles)))

(defmacro image (path &optional styles)
  `(make-node :type :image
              :content ,path
              :styles ,(expand-styles styles)))

;; Element creation
(defun create-element (node)
  "Create the GPU element for a node. Returns the ID or nil."
  (let ((styles (node-styles node)))
    (case (node-type node)
      (:rect
        (let ((id (make-rect)))
          (let ((c (getf styles :color)))    (when c (rect-color id c)))
          (let ((r (getf styles :radius)))   (when r (rect-radius id r)))
          (let ((bw (getf styles :border-width)))
            (when bw (rect-border id bw (or (getf styles :border-color) #x000000FF))))
          id))
      (:text
        (let* ((size (or (getf styles :size) 16))
               (lh (or (getf styles :line-height) (* size 1.4)))
               (id (make-text size lh)))
          (let ((f (or (getf styles :family) *default-font*)))
            (when f (text-family id f)))
          (let ((w (getf styles :weight))) (when w (text-weight id w)))
          (when (node-content node) (text-set id (node-content node)))
          (let ((c (getf styles :color))) (when c (text-color id c)))
          id))
      (:image
        (when (node-content node)
          (make-image (node-content node))))
      (otherwise nil))))

(defun create-elements (ui node)
  "Walk the tree, create GPU elements, register :name entries."
  (setf (node-id node) (create-element node))
  (let ((name (getf (node-styles node) :name)))
    (when name (setf (gethash name (ui-names ui)) node)))
  (dolist (child (node-children node))
    (create-elements ui child)))

;; Layout
(defun vertical-p (type)
  (member type '(:vstack :rect)))

(defun resolve-size (node axis available)
  "Resolve width (axis :w) or height (axis :h) for a node."
  (let ((sv (getf (node-styles node) (if (eq axis :w) :width :height)))
        (id (node-id node)))
    (cond
      ((numberp sv) sv)
      ((eq sv :fill) available)
      ((and (eq (node-type node) :text) id)
       (if (eq axis :w) (text-width id) (text-height id)))
      ((and (eq (node-type node) :image) id)
       (if (eq axis :w) (image-width id) (image-height id)))
      (t (if (eq axis :w) available nil)))))

(defun measure-node (node available-w available-h)
  "Compute width and height for a node and all descendants."
  (let* ((styles (node-styles node))
         (padding (or (getf styles :padding) 0))
         (gap (or (getf styles :gap) 0))
         (vertical (vertical-p (node-type node)))
         (w (resolve-size node :w available-w))
         (content-w (- w (* 2 padding)))
         (children (node-children node))
         (n (length children)))

    (setf (node-width node) w)

    (if vertical
        (dolist (child children)
          (measure-node child content-w (- available-h (* 2 padding))))
        (let* ((explicit-w (loop for c in children
                                 for sw = (getf (node-styles c) :width)
                                 when (numberp sw) sum sw))
               (total-gap (* gap (max 0 (1- n))))
               (remaining (- content-w explicit-w total-gap))
               (flex-count (count-if-not
                            (lambda (c) (numberp (getf (node-styles c) :width)))
                            children))
               (flex-w (if (> flex-count 0) (max 0 (/ remaining flex-count)) 0)))
          (dolist (child children)
            (let ((sw (getf (node-styles child) :width)))
              (measure-node child (if (numberp sw) sw flex-w)
                            (- available-h (* 2 padding)))))))

    (setf (node-height node)
          (or (resolve-size node :h available-h)
              (+ (* 2 padding)
                 (if children
                     (if vertical
                         (+ (loop for c in children sum (node-height c))
                            (* gap (max 0 (1- n))))
                         (loop for c in children maximize (node-height c)))
                     0))))))

(defun position-node (node x y)
  "Set x, y for a node and all descendants. Sizes must already be computed."
  (setf (node-x node) x (node-y node) y)

  (let* ((styles (node-styles node))
         (padding (or (getf styles :padding) 0))
         (gap (or (getf styles :gap) 0))
         (align (or (getf styles :align) :start))
         (justify (or (getf styles :justify) :start))
         (vertical (vertical-p (node-type node)))
         (content-w (- (node-width node) (* 2 padding)))
         (content-h (- (node-height node) (* 2 padding)))
         (children (node-children node))
         (n (length children))
         (children-main (if vertical
                            (loop for c in children sum (node-height c))
                            (loop for c in children sum (node-width c))))
         (main-avail (if vertical content-h content-w))
         (actual-gap (if (and (eq justify :between) (> n 1))
                         (/ (- main-avail children-main) (1- n))
                         gap))
         (main-offset (case justify
                        (:center (/ (- main-avail children-main
                                       (* gap (max 0 (1- n)))) 2))
                        (:end (- main-avail children-main
                                 (* gap (max 0 (1- n)))))
                        (otherwise 0)))
         (cx (+ x padding (if vertical 0 main-offset)))
         (cy (+ y padding (if vertical main-offset 0))))

    (dolist (child children)
      (let* ((cross-avail (if vertical content-w content-h))
             (cross-size (if vertical (node-width child) (node-height child)))
             (cross-offset (case align
                             (:center (/ (- cross-avail cross-size) 2))
                             (:end (- cross-avail cross-size))
                             (otherwise 0))))
        (if vertical
            (progn
              (position-node child (+ cx cross-offset) cy)
              (incf cy (+ (node-height child) actual-gap)))
            (progn
              (position-node child cx (+ cy cross-offset))
              (incf cx (+ (node-width child) actual-gap))))))))

(defun apply-layout (node)
  "Walk the tree, apply computed positions/sizes via FFI."
  (let ((id (node-id node)))
    (when id
      (let ((x (node-x node)) (y (node-y node))
            (w (node-width node)) (h (node-height node)))
        (case (node-type node)
          (:rect  (rect-position id x y) (rect-size id w h))
          (:text  (text-position id x y)
            (text-bounds id (truncate x) (truncate y)
                         (truncate (+ x w)) (truncate (+ y h))))
          (:image (image-position id x y) (image-size id w h))))))
  (dolist (child (node-children node))
    (apply-layout child)))

;; UI
(defun destroy-ui (ui)
  "Destroy all GPU elements in a UI tree."
  (labels ((walk (node)
             (when (node-id node)
               (case (node-type node)
                 (:rect  (rect-destroy (node-id node)))
                 (:text  (text-destroy (node-id node)))
                 (:image (image-destroy (node-id node)))))
             (dolist (child (node-children node))
               (walk child))))
    (walk (ui-root ui))))

(defun ui-ref (ui name)
  "Get the GPU element ID for a named element."
  (let ((node (gethash name (ui-names ui))))
    (when node (node-id node))))

(defun layout (ui)
  "Recompute layout and apply positions/sizes."
  (let* ((root (ui-root ui))
         (styles (node-styles root))
         (x (or (getf styles :x) 0))
         (y (or (getf styles :y) 0))
         (w (or (getf styles :width) (window-width)))
         (h (or (getf styles :height) (window-height))))
    (measure-node root w h)
    (position-node root x y)
    (apply-layout root)))

(defun build-ui (root)
  "Create GPU elements and compute layout for a node tree."
  (let ((ui (make-ui :root root :names (make-hash-table :test #'eq))))
    (create-elements ui root)
    (layout ui)
    ui))

(defmacro defui (name styles &body body)
  "Define a UI tree with full Lisp expressiveness."
  `(progn
     (when (boundp ',name) (destroy-ui (symbol-value ',name)))
     (defparameter ,name
       (build-ui (vstack ,styles ,@body)))))
