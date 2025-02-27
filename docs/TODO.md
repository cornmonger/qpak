todo!()
=======================================================================================================================

NOW
-----------------------------------------------------------------------------------------------------------------------
 - test & doc & cleanup
 - publish v1

ROADMAP
-----------------------------------------------------------------------------------------------------------------------
 - [qpak insert](#qpak-insert)
 - [qpak replace](#qpak-replace)
 - [qpak delete](#qpak-delete)
 
### qpak insert
`qpak insert [-f] <source_path> <pak_file> [pak_path_prefix]`

Inserts a file or directory into a PAK file. Appends to directories when forced. Maintains standard sort order. 

**source_path**: The path to the file or directory to insert   
**pak_file**: The PAK file to modify  
**pak_path_prefix**: The path prefix to use for the inserted item within the PAK. Default: (root)

Options:
- **-f --force**: Force overwriting existing items

### qpak replace
`qpak replace [-f] <source_path> <pak_file> <pak_path>`

Completely replaces a file or directory in a PAK file. Deletes existing directories when forced. Maintains standard
sort order. 

**source_path**: The path to the file or directory to insert   
**pak_file**: The PAK file to modify  
**pak_path**: The path prefix to use for the inserted item within the PAK. Default: (root)

Options:
- **-f --force**: Force deletion of existing items

### qpak delete
`qpak delete <pak_file> <pak_path>`

Deletes a file or directory in a PAK file. Maintains standard sort order. 

**pak_file**: The PAK file to modify  
**pak_path**: The path prefix to use for the inserted item within the PAK. Default: (root)
