Output argus.gif

Set FontSize 12
Set Width 1200
Set Height 600
Set LetterSpacing 0

Require tmux
Require curl
Require jq

Hide
  Type "tmux -f /dev/null -L test new-session -- bash" Enter
  Type "tmux split-window -d -h -p 38 -- bash && \" Enter
  Type "tmux set status && \" Enter
  Type 'tmux setw pane-border-style "fg=0" && \' Enter
  Type 'tmux setw pane-active-border-style "fg=0"' Enter
  Sleep 0.5
  Ctrl+L
  Sleep 1
Show

Type "# run argus, with a jq pipeline to pretty print!" Sleep 500ms Enter
Type "./argus | jq" Enter
Sleep 1

Ctrl+B
Type o

Sleep 1
Type "# send some requests!" Sleep 500ms Enter
Type "curl localhost:8080/test" Sleep 500ms Enter
Type "curl -XPOST --data 'post data!' -H 'secret: 12345' localhost:8080/submit" Sleep 500ms Enter
Sleep 8s
