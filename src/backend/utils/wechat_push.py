from datetime import timedelta
import json
from cachetools import TTLCache, cached
import httpx
import requests


class WechatMessagePush:
    def __init__(self, appid, appsecret, temple_id):
        self.appid = appid
        self.appsecret = appsecret

        # 模板id,参考公众号后面的模板消息接口 -> 模板ID(用于接口调用):IG1Kwxxxx
        self.temple_id = temple_id

        self.token = self.get_Wechat_access_token()

    @cached(TTLCache(maxsize=10, ttl=timedelta(minutes=10).total_seconds()))
    def get_Wechat_access_token(self):
        """
        获取微信的access_token： 获取调用接口凭证
        :return:
        """
        url = f"https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={self.appid}&secret={self.appsecret}"
        response = requests.get(url)

        res = response.json()
        print(res)
        if "access_token" in res:
            token = res["access_token"]
            return token

    def get_wechat_accout_fans_count(self):
        """
        获取微信公众号所有粉丝的openid
        """
        next_openid = ""
        url = f"https://api.weixin.qq.com/cgi-bin/user/get?access_token={self.token}&next_openid={next_openid}"
        response = requests.get(url)
        res = response.json()["data"]["openid"]
        return res

    async def send_wechat_temple_msg(self, content):
        """
        发送微信公众号的模板消息"""
        url = f"https://api.weixin.qq.com/cgi-bin/message/template/send?access_token={self.token}"

        fan_open_id = [
            "oS7qU6oB6CkwVZwJSiwJp6xXfE6o"
        ]  # self.get_wechat_accout_fans_count()
        for open_id in fan_open_id:
            body = {
                "touser": open_id,
                "template_id": self.temple_id,
                # 'url': 'http://www.jb51.net',
                "topcolor": "#667F00",
                "data": {"text": {"value": content}},
            }
            headers = {"Content-type": "application/json"}
            data = json.JSONEncoder().encode(body)
            async with httpx.AsyncClient() as client:
                res = await client.post(url=url, data=data, headers=headers)
                return res
