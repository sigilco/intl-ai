Output "docs/public/demo.gif"
Output "docs/public/intl-ai-demo.mp4"

Set Shell "bash"
Set FontSize 20
Set Width 1200
Set Height 650
Set Padding 20
Set Theme "Dracula"
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "REPO_ROOT=$(pwd) && source $REPO_ROOT/demo/setup-scratch.sh"
Enter
Type "clear"
Enter
Show

# 1. Title card
Type "bash $REPO_ROOT/demo/title-card.sh"
Enter
Wait+Screen /title-ready/
Sleep 1s
Type "clear"
Enter

# 2. Reveal source strings
Type "cat locales/en.json"
Enter
Sleep 2s
Type "clear"
Enter

# 3. Real check — shows missing translations
Type "intl-ai check"
Enter
Wait+Screen /\['de'\] Missing translations:/
Sleep 1s

# 4. Faked beat — gum HUD stands in for live LLM latency
Type "bash $REPO_ROOT/demo/translating-hud.sh"
Enter
Wait+Screen /hud-ready/
Sleep 0.5s
Type "clear"
Enter

# 5. Real fill — live colored progress + summary (real API call)
Type "intl-ai fill --concurrency 1"
Enter
Wait+Screen /Translated 9 keys across/
Sleep 2s
Type "clear"
Enter

# 6. Real check again — now clean
Type "intl-ai check"
Enter
Wait+Screen /\['de'\] All translations complete/
Sleep 1.5s
Type "clear"
Enter

# 7. Reveal translated content
Type "cat locales/es.json"
Enter
Sleep 2.5s
Type "clear"
Enter

# 8. Closing card
Type "bash $REPO_ROOT/demo/closing-card.sh"
Enter
Wait+Screen /closing-ready/
Sleep 2s
