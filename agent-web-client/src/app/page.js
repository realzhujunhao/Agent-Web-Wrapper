"use client"
import ChatComponent from "@/components/ChatComponent";
import Cookies from "js-cookie";
import { useEffect, useState } from "react";

async function fetchHistory(serverDomain, token) {
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
        return result.data
    } else {
        alert(`Error: fetch history server ${resp.err}`)
        return null;
    }
}

async function askAgent(serverDomain, token, query) {
    const resp = await fetch(`${serverDomain}/ask-agent`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            "Authorization": `Bearer ${token}`,
        },
        body: JSON.stringify({
            "message": query
        }),
    });
    if (!resp.ok) {
        alert(`Error: send message status ${resp.status}`);
        return;
    }
    const result = await resp.json();
    if (result.success) {
        return result.data
    } else {
        alert(`Error: send message server ${resp.err}`)
        return null;
    }
}

async function clearHistory(serverDomain, token) {
    const resp = await fetch(`${serverDomain}/clear-history`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            "Authorization": `Bearer ${token}`,
        },
        body: JSON.stringify(null),
    });
    if (!resp.ok) {
        alert(`Error: clear history status ${resp.status}`);
        return;
    }
    const result = await resp.json();
    if (result.success) {
        return result.data
    } else {
        alert(`Error: clear history server ${resp.err}`)
        return null;
    }
}

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
    const reloadHistory = async () => {
        console.log("debug: trigger reload")
        const data = await fetchHistory(serverDomain, token);
        console.log(`debug: messageList: ${JSON.stringify(messageList)}`)
        setMessageList(data);
    };
    useEffect(() => {
        if (token) {
            reloadHistory();
        }
    }, [token]);

    const sendMessage = async (query) => {
        console.log("debug: trigger sendMessage");
        await askAgent(serverDomain, token, query);
        reloadHistory();
    };

    const clearConversation = async () => {
        console.log("debug: trigger clearHistory");
        await clearHistory(serverDomain, token);
        reloadHistory();
    };

    // conditional rendering
    return (
        <>
            {
                token && messageList &&
                <ChatComponent
                    messageList={messageList}
                    sendMessage={sendMessage}
                    clearHistory={clearConversation}
                />
            }
        </>
    );
}
