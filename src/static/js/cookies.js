function cookiesConfirmed() {
    // 2
    $('#cookie-footer').hide();
    // 3
    var d = new Date();
    d.setTime(d.getTime() + (365 * 24 * 60 * 60 * 1000));
    var expires = "expires=" + d.toUTCString();
    // 4
    document.cookie = "cookies-accepted=true;" + expires;
}
