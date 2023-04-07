import http from 'k6/http';
export const options = {
    discardResponseBodies: true,
    scenarios: {
        post: {
            exec: 'POST',
            executor: 'constant-vus',
            vus: 2,
            duration: '30s',
        },
        get: {
            exec: 'GET',
            executor: 'constant-vus',
            vus: 10,
            duration: '30s',
        },
    },
};

export function POST() {
    const posturl = 'http://localhost:8080/orders';
    const orderId = Math.floor(Math.random() * 1000000);
    const payload = JSON.stringify({
        order_id: orderId,
        name: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse a risus scelerisque, interdum orci ac, malesuada odio. Cras lacinia ante sit amet risus rutrum fringilla. Nulla ligula felis, aliquet consectetur interdum nec, sagittis ut lacus. Suspendisse potenti. Curabitur in lectus nunc. Ut mollis efficitur leo, eget congue justo. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Integer faucibus, libero in porta convallis, eros erat commodo diam, sed fermentum sapien nulla vel tortor. Duis consectetur mauris auctor, imperdiet nisl et, feugiat enim. Fusce ac enim eu turpis blandit interdum. Suspendisse ultrices tortor nunc, ac volutpat felis porttitor vitae. Vivamus venenatis, turpis et fringilla iaculis, velit diam bibendum felis, at mollis urna nisi tincidunt dolor. Fusce tincidunt pulvinar viverra.",
        transport_id: Math.floor(Math.random() * 100000)
    });

    const params = {
        headers: {
            'Content-Type': 'application/json',
        },
    };

    http.post(posturl, payload, params);
}
export function GET() {
    const orderId = Math.floor(Math.random() * 100000);
    const geturl = `http://localhost:8080/orders/${orderId}`;
    http.get(geturl);
}