export function getRandomStringColor(s: string, saturation = 90, lightness = 60) {
    // return 'black';
    // console.log(s);
    let h =  360-1337;
    for(let i = 0; i < s.length; i++){
        h = Math.imul(31, h) + (s.charCodeAt(i)) | 0;
    }

    return `hsl(${(h % 360)}, ${saturation}%, ${lightness}%)`;
}

