d = .directory~new
d~setMethod("go", "do f over .context~stackframes; say f~type f~name f~line; end; return 1")
say d~go
d2 = .dd~new
d2~setMethod("go", "do f over .context~stackframes; say f~type f~name f~line; end; return 2")
say d2~go
::class dd subclass directory
