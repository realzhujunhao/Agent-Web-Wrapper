"use client"
import ChatComponent from "@/components/ChatComponent";
import Cookies from "js-cookie";
import { useEffect, useState } from "react";

export default function Home() {
    const serverDomain = "http://localhost:8085";
    const [token, setToken] = useState(null);

    // session
    const testAndSetToken = async () => {
        const existingToken = Cookies.get("jwt");
        if (existingToken && existingToken != token) {
            setToken(existingToken);
            console.log("debug: cookie exists");
            console.log(existingToken);
            // reset expire time
            Cookies.set("jwt", existingToken, { expires: 30 });
        } else {
            console.log("debug: cookie does not exist");
            const resp = await fetch(`${serverDomain}/init-session`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify(null),
            });
            if (!resp.ok) {
                alert(`Error: Init session response status ${resp.status}`);
                return;
            }
            const result = await resp.json();
            console.log("debug: receive response from init-session");
            console.log(result);

            if (result.success) {
                Cookies.set("jwt", result.data, { expires: 30 });
                setToken(result.data);
                console.log("debug: set cookie")
                console.log(result.data);
            } else {
                // unreachable!
                alert(`Error: Init session server error ${result.err}`);
            }
        }
    };

    useEffect(() => {
        testAndSetToken();
    }, []);

    useEffect(() => {
        console.log(`debug: now token is ${token}`);
    }, [token]);


    const [messageList, setMessageList] = useState(null);
    // message list
    const loadChatHistory = async () => {
        const resp = await fetch(`${serverDomain}/fetch-history`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                "Authorization": `Bearer ${token}`,
            },
            body: JSON.stringify(null),
        });
        if (!resp.ok) {
            alert(`Error: fetch history status ${resp.status}`);
            return;
        }
        const result = await resp.json();
        if (result.success) {
            setMessageList(result.data)
            console.log("debug: receive message list")
            console.log(result.data);
        }
    };

    useEffect(() => {
        if (token) {
            loadChatHistory();
        }
    }, [token]);

    // conditional rendering
    return (
        <>
            {
                token && messageList &&
                <ChatComponent
                    messageList={messageList}
                />
            }
        </>
    );
}
