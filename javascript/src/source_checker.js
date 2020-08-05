const fs = require('fs');

module.exports = {
    checkSource: function (source_file) { 
        if (!fs.existsSync(source_file)) {
            throw new Error(`File ${source_file} not found`); 
        } else if (source_file.split('.').slice(-1)[0] != 'js') {
            throw new Error(`Input ${source_file} is not a javascript file`);
        }
        
        return source_file; 
    }
}
