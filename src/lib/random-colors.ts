export function getRandomStringColor(s: string, saturation = 75, lightness = 50) {
    // return '#292d2a';
    // console.log(s);
    let h =  360-1337;
    for(let i = 0; i < s.length; i++){
        h = Math.imul(31, h) + (s.charCodeAt(i)) | 0;
    }

    return `hsl(${(h % 360)}, ${saturation}%, ${lightness}%)`;
}

