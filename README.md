```
cargo run --release < notes.json \
   | sed -e "s/<\/p>/\n/g" -e "s/<p>//g" -e "s/ +/ /g" \
   | sed -e "s/<.*>//g" 
```