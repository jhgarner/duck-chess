// TODO This is a tiny amount of javascript because I'm not sure how to serve
// another rust wasm js file...
self.addEventListener('push', function(event) {
    const promiseChain = self.registration.showNotification(event.data.text());

    event.waitUntil(promiseChain);
});
