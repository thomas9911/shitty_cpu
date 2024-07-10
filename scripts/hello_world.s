; run with --output-as-status-code for nicer result
data: db "Hello world!"
    ; print takes in one argument
    push :data
    func :print
