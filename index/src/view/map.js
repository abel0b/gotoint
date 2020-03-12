function (doc) {
    let buffer = '';
    for (let idx=0; idx < doc.extract.length; idx++) {
        if (doc.extract[idx] == ' ' || idx == doc.extract.length-1) {
            emit(buffer, [doc._id, idx]);
            buffer = '';
        }
        else {
            buffer = buffer + doc.extract[idx]; 
        }
    }
}