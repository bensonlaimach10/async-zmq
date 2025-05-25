use async_zmq::{Result, StreamExt};

#[async_std::test]
async fn test_pub_sub_watermarks() -> Result<()> {
    // Create publisher with watermark
    let publisher = async_zmq::publish("tcp://127.0.0.1:*")?
        .bind()?
        .set_send_hwm(100)?;
    
    let endpoint = publisher.as_raw_socket().get_last_endpoint()?.unwrap();

    // Create subscriber with watermark
    let subscriber = async_zmq::subscribe(&endpoint)?
        .connect()?
        .set_receive_hwm(100)?;

    // Verify watermark settings
    assert_eq!(publisher.get_send_hwm()?, 100);
    assert_eq!(subscriber.get_receive_hwm()?, 100);

    Ok(())
}

#[async_std::test]
async fn test_req_rep_watermarks() -> Result<()> {
    // Create reply socket with watermarks
    let reply: &async_zmq::Reply<_, _> = async_zmq::reply("tcp://127.0.0.1:*")?
        .bind()?
        .set_receive_hwm(100)?
        .set_send_hwm(200)?;
    
    let endpoint = reply.as_raw_socket().get_last_endpoint()?.unwrap();

    // Create request socket with watermarks
    let request = async_zmq::request(&endpoint)?
        .connect()?
        .set_receive_hwm(100)?
        .set_send_hwm(200)?;

    // Verify watermark settings
    assert_eq!(reply.get_receive_hwm()?, 100);
    assert_eq!(reply.get_send_hwm()?, 200);
    assert_eq!(request.get_receive_hwm()?, 100);
    assert_eq!(request.get_send_hwm()?, 200);

    Ok(())
} 