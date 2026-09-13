-- extension: ipv4 validator

local function has_valid_characters(text)
    for i = 1, #text do
        local character = text:sub(i, i)
        local is_digit = character >= "0" and character <= "9"
        if not is_digit and character ~= "." then
            return false
        end
    end
    return true
end

local function has_valid_structure(text)
    if not has_valid_characters(text) then
        return false
    end
    if text:sub(1, 1) == "." or text:sub(-1) == "." then
        return false
    end
    if text:find("..", 1, true) ~= nil then
        return false
    end
    return true
end

local function split_octets(text)
    local octets = {}
    for part in text:gmatch("[^.]+") do
        table.insert(octets, part)
    end
    return octets
end

local function to_binary_byte(number)
    local bits = {}
    for i = 7, 0, -1 do
        local bit_value = 2 ^ i
        if number >= bit_value then
            table.insert(bits, "1")
            number = number - bit_value
        else
            table.insert(bits, "0")
        end
    end
    return table.concat(bits)
end

return {
    prefix = "ip_",

    add = function(key, value)
        if not has_valid_structure(value) then
            error("endereco ip deve conter apenas digitos separados por pontos", 0)
        end

        local octets = split_octets(value)
        if #octets ~= 4 then
            error("endereco ip deve ter exatamente 4 octetos separados por ponto", 0)
        end

        local numbers = {}
        for i, text in ipairs(octets) do
            if #text > 3 then
                error("octeto '" .. text .. "' deve ter no maximo 3 digitos", 0)
            end
            local number = tonumber(text)
            if number > 255 then
                error("octeto '" .. text .. "' fora da faixa permitida (0 a 255)", 0)
            end
            numbers[i] = number
        end

        return numbers[1] .. "." .. numbers[2] .. "." .. numbers[3] .. "." .. numbers[4]
    end,

    get = function(key, value)
        local octets = split_octets(value)
        local binary_octets = {}
        for i, text in ipairs(octets) do
            binary_octets[i] = to_binary_byte(tonumber(text))
        end
        return table.concat(binary_octets, ".")
    end,
}
