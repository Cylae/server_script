import re

file_path = 'server_manager/src/core/users.rs'
with open(file_path, 'r') as f:
    content = f.read()

# Replace bcrypt::verify(password, hash_str).unwrap_or(false)
content = content.replace(
    'return bcrypt::verify(password, hash_str).unwrap_or(false);',
    '''match bcrypt::verify(password, hash_str) {
            Ok(valid) => return valid,
            Err(e) => {
                log::warn!("bcrypt verification error: {}", e);
                return false;
            }
        }'''
)

# And similarly in verify_and_migrate
content = content.replace(
    'else if hash_str.starts_with("$2") && bcrypt::verify(password, &hash_str).unwrap_or(false)',
    '''else if hash_str.starts_with("$2") && {
            match bcrypt::verify(password, &hash_str) {
                Ok(valid) => valid,
                Err(e) => {
                    log::warn!("bcrypt verification error during migration check: {}", e);
                    false
                }
            }
        }'''
)

with open(file_path, 'w') as f:
    f.write(content)
