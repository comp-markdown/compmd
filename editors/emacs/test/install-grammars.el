;;; install-grammars.el --- CompMD's three grammars installed as a writer installs them -*- lexical-binding: t; -*-

;;; Commentary:

;; `treesit-install-language-grammar' on each of the mode's recipes, from comp-markdown/compmd-grammars, into a
;; folder, in a batch Emacs (it needs git and a C compiler):
;;
;;   emacs --batch -l test/install-grammars.el <folder>
;;
;; It exits 0 when the three load from the folder, else 1.

;;; Code:

(load (expand-file-name "../compmd-ts-mode.el" (file-name-directory load-file-name)) nil t)

(let ((folder (expand-file-name (pop command-line-args-left)))
      (missing nil))
  (setq treesit-extra-load-path (list folder))
  (dolist (language '(compmd compmd-inline cel))
    (treesit-install-language-grammar language folder)
    (unless (treesit-language-available-p language)
      (push language missing)))
  (if missing
      (progn (princ (format "not installed: %S\n" missing)) (kill-emacs 1))
    (princ (format "compmd, compmd-inline and cel installed in %s\n" folder))
    (kill-emacs 0)))

;;; install-grammars.el ends here
